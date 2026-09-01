using System;
using System.Threading;
using MediatR.Licensing;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;
using Shouldly;
using Xunit;

namespace MediatR.Tests.Licensing;

// Shares the non-parallel ServiceFactory collection because it mutates the static
// MediatRServiceCollectionExtensions.LicenseChecked flag.
[Collection(nameof(ServiceFactoryCollectionBehavior))]
public class LicenseValidationBackgroundTests
{
    // Regression test for the license-validation deadlock (AutoMapper #4640, same root cause):
    // validation must not run on the Mediator construction thread. Constructing a Mediator under
    // a lazily-built DI singleton holds the container's build lock, and validating there could
    // deadlock the whole app under a cold-start thread-pool starvation. Validation is logging-only,
    // so it is offloaded to a dedicated background thread. Rather than clamp the global thread pool
    // (flaky, process-wide), we assert the property that makes the deadlock impossible: constructing
    // the Mediator returns without validating, and validation logs on a *different* thread.
    [Fact]
    public void License_validation_runs_off_the_construction_thread()
    {
        MediatRServiceCollectionExtensions.LicenseChecked = false;

        var loggerProvider = new ThreadCapturingLoggerProvider();

        var services = new ServiceCollection();
        services.AddLogging();
        services.AddSingleton<ILoggerProvider>(loggerProvider);
        services.AddTransient<IMediator, Mediator>();
        // A non-empty (junk) key forces the validation path to run; its result is logged.
        services.AddSingleton(new MediatRServiceConfiguration { LicenseKey = "not-a-real-license-key" });
        services.AddSingleton<LicenseAccessor>();
        services.AddSingleton<LicenseValidator>();

        var container = services.BuildServiceProvider();

        var constructingThreadId = Environment.CurrentManagedThreadId;

        _ = container.GetRequiredService<IMediator>(); // constructs Mediator -> CheckLicense

        // Construction returned without blocking on validation; the license log should arrive
        // shortly, on a thread other than the one that constructed the Mediator.
        loggerProvider.LicenseLogged.Wait(TimeSpan.FromSeconds(5)).ShouldBeTrue();
        loggerProvider.LoggingThreadId.ShouldNotBe(constructingThreadId);
    }

    private sealed class ThreadCapturingLoggerProvider : ILoggerProvider
    {
        public readonly ManualResetEventSlim LicenseLogged = new(false);
        public int LoggingThreadId;

        public ILogger CreateLogger(string categoryName) =>
            categoryName == "LuckyPennySoftware.MediatR.License"
                ? new CapturingLogger(this)
                : NullLogger.Instance;

        public void Dispose() => LicenseLogged.Dispose();

        private sealed class CapturingLogger : ILogger
        {
            private readonly ThreadCapturingLoggerProvider _owner;

            public CapturingLogger(ThreadCapturingLoggerProvider owner) => _owner = owner;

            public IDisposable? BeginScope<TState>(TState state) where TState : notnull => null;

            public bool IsEnabled(LogLevel logLevel) => true;

            public void Log<TState>(LogLevel logLevel, EventId eventId, TState state, Exception? exception,
                Func<TState, Exception?, string> formatter)
            {
                _owner.LoggingThreadId = Environment.CurrentManagedThreadId;
                _owner.LicenseLogged.Set();
            }
        }
    }
}
