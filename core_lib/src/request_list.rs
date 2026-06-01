#[macro_export]
macro_rules! call_macro_with_request_list {
    ($macro:ident) => {
        $macro! {
            get_categories() -> Vec<core_lib::Category>;
            get_assets() -> Vec<core_lib::Asset>;
            get_latest_record() -> Option<core_lib::AllocationRecord>;
            get_alloc_diagram_data(core_lib::GetAllocDiagramDataArgs) -> core_lib::AllocationDiagramData;
            add_asset(core_lib::AddAssetArgs) -> ();
        }
    };
}
