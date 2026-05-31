#[macro_export]
macro_rules! call_macro_with_request_list {
    ($macro:ident) => {
        $macro! {
            get_categories() -> Vec<Category>;
            get_assets() -> Vec<Asset>;
            get_latest_record() -> Option<AllocationRecord>;
            get_alloc_diagram_data(GetAllocDiagramDataArgs) -> AllocationDiagramData;
            add_asset(AddAssetArgs) -> ();
        }
    };
}
