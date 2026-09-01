using MediatR.Licensing;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Testing;
using Shouldly;
using Xunit;

namespace MediatR.Tests.Licensing;

public class LicenseAccessorTests
{
    [Theory]
    [InlineData(null)]
    [InlineData("")]
    [InlineData("   ")]
    [InlineData("\t")]
    public void Should_return_unconfigured_license_for_missing_or_blank_key(string? key)
    {
        var factory = new LoggerFactory();
        var provider = new FakeLoggerProvider();
        factory.AddProvider(provider);

        var config = new MediatRServiceConfiguration { LicenseKey = key };
        var accessor = new LicenseAccessor(config, factory);

        var license = accessor.Current;

        license.IsConfigured.ShouldBeFalse();

        var logMessages = provider.Collector.GetSnapshot();
        logMessages.ShouldNotContain(log => log.Level == LogLevel.Critical);
    }

    [Fact]
    public void Should_return_unconfigured_license_when_static_key_is_blank()
    {
        var factory = new LoggerFactory();
        var provider = new FakeLoggerProvider();
        factory.AddProvider(provider);

        var original = Mediator.LicenseKey;
        try
        {
            Mediator.LicenseKey = "   ";
            var accessor = new LicenseAccessor(factory);

            var license = accessor.Current;

            license.IsConfigured.ShouldBeFalse();

            var logMessages = provider.Collector.GetSnapshot();
            logMessages.ShouldNotContain(log => log.Level == LogLevel.Critical);
        }
        finally
        {
            Mediator.LicenseKey = original;
        }
    }
}
