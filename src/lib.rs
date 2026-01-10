pub mod config;
pub mod tsconfig;
pub mod parser;
pub mod type_mapper;
pub mod generator;
pub mod cache;
pub mod scanner;
pub mod table_registry;
pub mod table_mapping;
pub mod validator;
pub mod ts_generator;

pub use table_registry::TableRegistry;
pub use table_mapping::TableMappingResolver;
pub use ts_generator::TsCodeGenerator;
