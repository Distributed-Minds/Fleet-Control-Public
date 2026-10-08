// GENERATED. DO NOT EDIT.
// generator: l2-adapter-shim-generator@0
// descriptor-sha256: 29663431523e65018681c1644020750af03edc4d82bf478c97966960da0981a5
#nullable enable
namespace Signet.Research;

public interface ISignetAdapterShim
{
    /// <summary>Version of the game/engine integration surface exposed by this shim.</summary>
    uint SurfaceVersion();
    /// <summary>Read one local control by adapter-defined identifier.</summary>
    float? CaptureControl(string controlId);
    /// <summary>Read one calibration metric when the legitimate integration surface exposes it.</summary>
    double? ObserveMetric(string metricId);
    /// <summary>Present one already-resolved local appearance choice.</summary>
    bool PresentChoice(string choiceId, string payloadJson);
}
