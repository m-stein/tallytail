#[macro_export]
macro_rules! call_macro_with_request_list {
    ($macro:ident) => {
        $macro! {
            get_categories() -> Vec<core_lib::Category>;
            get_assets() -> Vec<core_lib::Asset>;
            get_latest_record() -> Option<core_lib::AllocationRecord>;
            get_alloc_diagram_data(core_lib::GetAllocDiagramDataArgs) -> core_lib::AllocationDiagramData;
            add_asset(core_lib::AddAssetArgs) -> ();
            log_transaction(core_lib::LogTransactionInput) -> ();
            list_transactions() -> Vec<core_lib::ListedTransaction>;
            list_portfolio_overview_items() -> Vec<core_lib::PortfolioOverviewItem>;
            list_portfolio_isin_items(String) -> Vec<core_lib::PortfolioIsinItem>;
            load_png_data(String) -> Vec<u8>;
            configure_categories(core_lib::ConfigureCatgoriesInput) -> (core_lib::ConfigureCatgoriesInput, Option<String>);
        }
    };
}
