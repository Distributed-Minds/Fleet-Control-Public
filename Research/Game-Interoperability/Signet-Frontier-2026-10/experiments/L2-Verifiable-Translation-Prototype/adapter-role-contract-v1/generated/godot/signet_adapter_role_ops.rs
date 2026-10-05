// GENERATED. DO NOT EDIT.
// generator: l2-role-contract-generator@1
// descriptor-sha256: 2c3b3d9f374172435ec505358e9fc084b844e0914fd2bd77b12f6aacaebd12e2

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterOperation {
    HostSurfaceVersion,
}

pub trait SignetAdapterRoleOps {
    /// role=HOST; authority=INTEGRATION_EVIDENCE; evidence=HARNESS
    fn host_surface_version(&mut self, request: &[u8], response: &mut Vec<u8>) -> Result<(), i32>;
}
