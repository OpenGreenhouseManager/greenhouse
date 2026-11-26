// @generated automatically by Diesel CLI.

diesel::table! {
    device (id) {
        id -> Uuid,
        name -> Varchar,
        address -> Varchar,
        description -> Varchar,
        canscript -> Bool,
        scraping -> Bool,
    }
}

diesel::table! {
    device_key (key) {
        key -> Text,
        device_id -> Uuid,
    }
}

diesel::joinable!(device_key -> device (device_id));

diesel::allow_tables_to_appear_in_same_query!(device, device_key,);
