mod delete_entry;
mod get_entries;
mod get_entry;
mod post_entry;

pub(crate) use delete_entry::exec as delete_entry_route;
pub(crate) use get_entries::exec as get_entries_route;
pub(crate) use get_entry::exec as get_entry_route;
pub(crate) use post_entry::exec as post_entry_route;
