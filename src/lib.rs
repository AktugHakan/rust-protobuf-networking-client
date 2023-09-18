// include!(concat!(env!("OUT_DIR"), "/snazzy.items.rs"));

/*
Include the `items` module, which is generated from items.proto.
It is important to maintain the same structure as in the proto.
pub mod snazzy {
    pub mod items {
        include!(concat!(env!("OUT_DIR"), "/snazzy.items.rs"));
    }
}

use snazzy::items;

pub fn create_large_shirt(color: String) -> items::Shirt {
    let mut shirt = items::Shirt::default();
    shirt.color = color;
    shirt.set_size(items::shirt::Size::Large);
    shirt
}
*/

pub mod cmd_io;
pub mod network;
pub mod proto;

pub const MB: u64 = 1024 * 1024;

pub mod protocom {
    pub mod request {
        include!(concat!(env!("OUT_DIR"), "/protocom.request.rs"));
    }

    pub mod response {
        include!(concat!(env!("OUT_DIR"), "/protocom.response.rs"));
    }
}
