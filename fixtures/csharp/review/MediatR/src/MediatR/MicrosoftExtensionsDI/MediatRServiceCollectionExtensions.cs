using System;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using MediatR;
using MediatR.Licensing;
using MediatR.Pipeline;
using MediatR.Registration;
using Microsoft.Extensions.Logging;

namespace Microsoft.Extensions.DependencyInjection;

/// <summary>
/// Extensions to scan for MediatR handlers and registers them.
/// - Scans for any handler interface implementations and registers them as <see cref="ServiceLifetime.Transient"/>
/// - Scans for any <see cref="IRequestPreProcessor{TRequest}"/> and <see cref="IRequestPostProcessor{TRequest,TResponse}"/> implementations and registers them as transient instances
/// Registers <see cref="IMediator"/> as a transient instance
/// After calling AddMediatR you can use the container to resolve an <see cref="IMediator"/> instance.
/// This does not scan for any <see cref="IPipelineBehavior{TRequest,TResponse}"/> instances including <see cref="RequestPreProcessorBehavior{TRequest,TResponse}"/> and <see cref="RequestPreProcessorBehavior{TRequest,TResponse}"/>.
/// To register behaviors, use the <see cref="ServiceCollectionServiceExtensions.AddTransient(IServiceCollection,Type,Type)"/> with the open generic or closed generic types.
/// </summary>
public static class MediatRServiceCollectionExtensions
{
    /// <summary>
    /// Registers handlers and mediator types from the specified assemblies
    /// </summary>
    /// <param name="services">Service collection</param>
    /// <param name="configuration">The action used to configure the options</param>
    /// <returns>Service collection</returns>
    public static IServiceCollection AddMediatR(this IServiceCollection services, 
        Action<MediatRServiceConfiguration> configuration)
    {
        var serviceConfig = new MediatRServiceConfiguration();

        configuration.Invoke(serviceConfig);

        return services.AddMediatR(serviceConfig);
    }
    
    /// <summary>
    /// Registers handlers and mediator types from the specified assemblies
    /// </summary>
    /// <param name="services">Service collection</param>
    /// <param name="configuration">Configuration options</param>
    /// <returns>Service collection</returns>
    public static IServiceCollection AddMediatR(this IServiceCollection services, 
        MediatRServiceConfiguration configuration)
    {
        if (!configuration.AssembliesToRegister.Any())
        {
            throw new ArgumentException("No assemblies found to scan. Supply at least one assembly to scan for handlers.");
        }

        ServiceRegistrar.SetGenericRequestHandlerRegistrationLimitations(configuration);

        ServiceRegistrar.AddMediatRClassesWithTimeout(services, configuration);

        ServiceRegistrar.AddRequiredServices(services, configuration);

        return services;
    }
    
    internal static void CheckLicense(this IServiceProvider serviceProvider)
    {
        if (LicenseChecked)
        {
            return;
        }

        // Resolve the license services synchronously so a missing registration still surfaces
        // on the caller. The resolutions are cheap; only the JWT validation below is expensive.
        var licenseAccessor = serviceProvider.GetRequiredService<LicenseAccessor>();
        var licenseValidator = serviceProvider.GetRequiredService<LicenseValidator>();

        LicenseChecked = true;

        // License validation is logging-only — it gates no Mediator behavior. Running it on the
        // Mediator construction path is unsafe: under a lazily-built DI singleton the container
        // holds its build lock, and a cold-start thread-pool starvation then deadlocks the whole
        // app (same root cause as AutoMapper #4640, which used Task.Run(...).GetResult() under a
        // singleton-build lock). Offload the JWT validation to a dedicated background thread.
        _ = Task.Factory.StartNew(
            () => ValidateLicense(serviceProvider, licenseAccessor, licenseValidator),
            CancellationToken.None,
            TaskCreationOptions.LongRunning, // dedicated thread, never the (possibly starved) pool
            TaskScheduler.Default);          // honor LongRunning regardless of the ambient scheduler
    }

    private static void ValidateLicense(IServiceProvider serviceProvider, LicenseAccessor licenseAccessor, LicenseValidator licenseValidator)
    {
        try
        {
            var license = licenseAccessor.Current;
            licenseValidator.Validate(license);
        }
        catch (Exception ex)
        {
            // Never let a fire-and-forget failure surface as an unobserved task exception.
            serviceProvider.GetService<ILoggerFactory>()?
                .CreateLogger("LuckyPennySoftware.MediatR.License")
                .LogError(ex, "Error validating the Lucky Penny software license key");
        }
    }

    internal static bool LicenseChecked { get; set; }
}