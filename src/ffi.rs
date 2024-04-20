// TODO: Any better way to link a windows library?
#[cfg(windows)]
#[link(name = "Iphlpapi")]
extern "C" {}

extern "C" {
    pub fn getdefaultgateway(addr: *mut u32) -> i32;
}
