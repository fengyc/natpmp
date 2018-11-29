// TODO: Any better way to link a windows library?
#[cfg(windows)]
#[link(name = "Iphlpapi")]
extern "C" {}

extern "C" {
    pub static RS_EWOULDBLOCK: i32;
    pub static RS_ECONNREFUSED: i32;
    pub fn getdefaultgateway(addr: *mut u32) -> i32;
}
