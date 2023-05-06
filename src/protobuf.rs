pub mod securemessage {
    include!(concat!(env!("OUT_DIR"), "/securemessage.rs"));
}
pub mod securegcm {
    include!(concat!(env!("OUT_DIR"), "/securegcm.rs"));
}
pub mod location {
    pub mod nearby {
        pub mod connections {
            include!(concat!(env!("OUT_DIR"), "/location.nearby.connections.rs"));
        }
    }
}
pub mod sharing {
    pub mod nearby {

        include!(concat!(env!("OUT_DIR"), "/sharing.nearby.rs"));
    }
}
