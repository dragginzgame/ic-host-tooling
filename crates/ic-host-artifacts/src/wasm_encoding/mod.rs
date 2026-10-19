//! Core Wasm32 modules with fixed memory and exported, call-only functions.
//!
//! `wasm-encoder` owns binary encoding. Callers supply every import/export name,
//! memory size, data placement and call. No IC entry-point or reply policy is chosen.

use std::{collections::BTreeSet, fmt};
use wasm_encoder::{
    CodeSection, ConstExpr, DataSection, EntityType, ExportKind, ExportSection, Function,
    FunctionSection, ImportSection, Instruction, MemorySection, MemoryType, Module, TypeSection,
    ValType,
};

/// One imported function, taking only i32 arguments and returning nothing.
#[derive(Clone, Copy, Debug)]
pub struct FunctionImport<'a> {
    /// Exact import module name.
    pub module: &'a str,
    /// Exact imported field name.
    pub name: &'a str,
    /// Number of i32 parameters.
    pub parameters: u32,
}

/// One call to an imported function, with constant i32 arguments.
#[derive(Clone, Debug)]
pub struct Call {
    /// Index into the module's import list.
    pub import: u32,
    /// Arguments in parameter order; raw i32 bit patterns are preserved.
    pub arguments: Vec<i32>,
}

/// A () -> () function that makes the selected calls in order.
#[derive(Clone, Debug)]
pub struct ExportedFunction<'a> {
    /// Exact function export name; no prefix or normalization is added.
    pub name: &'a str,
    /// Calls in execution order. The encoder supplies the final End instruction.
    pub calls: Vec<Call>,
}

/// One nonshared Wasm32 memory whose minimum equals its maximum.
#[derive(Clone, Copy, Debug)]
pub struct Memory32<'a> {
    /// Number of standard 64-KiB pages, from zero through 65,536.
    pub pages: u32,
    /// Exact memory export name.
    pub export: &'a str,
}

/// Bytes initialized in memory zero before any exported function runs.
#[derive(Clone, Copy, Debug)]
pub struct DataSegment<'a> {
    /// Unsigned Wasm32 address; the matching i32 constant preserves these bits.
    pub offset: u32,
    /// Exact bytes, without interpretation. Empty and overlapping segments are allowed.
    pub bytes: &'a [u8],
}

/// Inputs for a core module with fixed memory and call-only exported functions.
#[derive(Clone, Copy, Debug)]
pub struct CallModule<'a> {
    /// Function imports, defining the indices used by calls.
    pub imports: &'a [FunctionImport<'a>],
    /// Exported () -> () functions, following the imported function index space.
    pub functions: &'a [ExportedFunction<'a>],
    /// The single fixed memory and its export name.
    pub memory: Memory32<'a>,
    /// Active memory-zero segments in initialization order.
    pub data: &'a [DataSegment<'a>],
}

/// Encoding refused without returning a partial module.
#[derive(Debug, Eq, PartialEq)]
pub enum EncodingError {
    /// A count or complete encoded extent exceeds this API's u32 size domain.
    LengthOverflow,
    /// The complete encoded module exceeds the output-byte allowance.
    LimitExceeded {
        /// Caller-selected maximum complete output length.
        limit: usize,
    },
    /// Fixed Wasm32 memory exceeds 65,536 standard pages.
    InvalidMemory,
    /// A call references an absent import or has the wrong argument count.
    InvalidCall {
        /// Exported-function position.
        function: usize,
        /// Call position within that function.
        call: usize,
    },
    /// Function/memory export names are not unique.
    DuplicateExport,
    /// An active data segment exceeds the selected initial memory extent.
    DataOutOfBounds {
        /// Input segment position.
        segment: usize,
    },
}

impl fmt::Display for EncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthOverflow => f.write_str("Wasm encoding extent exceeds u32"),
            Self::LimitExceeded { limit } => write!(f, "encoded Wasm exceeds {limit} bytes"),
            Self::InvalidMemory => f.write_str("fixed Wasm32 memory exceeds 65536 pages"),
            Self::InvalidCall { function, call } => {
                write!(f, "invalid import call {call} in function {function}")
            }
            Self::DuplicateExport => f.write_str("duplicate Wasm export name"),
            Self::DataOutOfBounds { segment } => {
                write!(f, "data segment {segment} exceeds initial memory")
            }
        }
    }
}
impl std::error::Error for EncodingError {}

/// Encode a fixed-memory module whose exported functions call supplied imports.
///
/// Uses the private upstream encoder for types, sections, instructions and data.
/// Only i32 arguments and void results are supported. No raw opcode escape hatch,
/// start function, execution, IC naming or installation/lifecycle policy is supplied.
/// Inputs keep their order and exact names/bytes. Overlapping segments initialize
/// in order; callers own data layout and the semantics of imported functions.
///
/// The allowance bounds returned bytes, not peak heap or CPU use. Obvious input
/// lower bounds are checked before constructing sections; final framing is checked
/// after upstream encoding. Allocation failures follow the upstream Vec contract.
/// The complete output is limited to u32 bytes, independently of host pointer size.
///
/// # Errors
/// Returns length/output admission, memory, import-call, export or data-extent refusal.
pub fn encode(module: &CallModule<'_>, max_bytes: usize) -> Result<Vec<u8>, EncodingError> {
    admit(module, max_bytes)?;
    let import_count = length(module.imports.len())?;
    let void_type = module
        .imports
        .iter()
        .position(|import| import.parameters == 0)
        .map_or(Ok(import_count), length)?;
    let mut types = TypeSection::new();
    let mut imports = ImportSection::new();
    for (index, import) in module.imports.iter().enumerate() {
        types
            .ty()
            .function((0..import.parameters).map(|_| ValType::I32), []);
        imports.import(
            import.module,
            import.name,
            EntityType::Function(length(index)?),
        );
    }
    if void_type == import_count {
        types.ty().function([], []);
    }
    let mut functions = FunctionSection::new();
    let mut exports = ExportSection::new();
    let mut code = CodeSection::new();
    exports.export(module.memory.export, ExportKind::Memory, 0);
    for (index, function) in module.functions.iter().enumerate() {
        functions.function(void_type);
        exports.export(
            function.name,
            ExportKind::Func,
            import_count + length(index)?,
        );
        let mut body = Function::new([]);
        for call in &function.calls {
            for &argument in &call.arguments {
                body.instruction(&Instruction::I32Const(argument));
            }
            body.instruction(&Instruction::Call(call.import));
        }
        body.instruction(&Instruction::End);
        code.function(&body);
    }
    let mut memory = MemorySection::new();
    memory.memory(MemoryType {
        minimum: u64::from(module.memory.pages),
        maximum: Some(u64::from(module.memory.pages)),
        memory64: false,
        shared: false,
        page_size_log2: None,
    });
    let mut data = DataSection::new();
    for segment in module.data {
        let address = i32::from_le_bytes(segment.offset.to_le_bytes());
        data.active(
            0,
            &ConstExpr::i32_const(address),
            segment.bytes.iter().copied(),
        );
    }
    let mut encoded = Module::new();
    encoded
        .section(&types)
        .section(&imports)
        .section(&functions)
        .section(&memory)
        .section(&exports)
        .section(&code)
        .section(&data);
    let bytes = encoded.finish();
    length(bytes.len())?;
    if bytes.len() > max_bytes {
        return Err(EncodingError::LimitExceeded { limit: max_bytes });
    }
    Ok(bytes)
}

fn length(value: usize) -> Result<u32, EncodingError> {
    u32::try_from(value).map_err(|_| EncodingError::LengthOverflow)
}

fn charge(total: &mut usize, amount: usize, limit: usize) -> Result<(), EncodingError> {
    *total = total
        .checked_add(amount)
        .filter(|value| *value <= limit)
        .ok_or(EncodingError::LimitExceeded { limit })?;
    Ok(())
}

fn admit(module: &CallModule<'_>, limit: usize) -> Result<(), EncodingError> {
    let mut minimum = 0;
    charge(&mut minimum, 8, limit)?;
    length(
        module
            .imports
            .len()
            .checked_add(1)
            .ok_or(EncodingError::LengthOverflow)?,
    )?;
    length(
        module
            .functions
            .len()
            .checked_add(module.imports.len())
            .ok_or(EncodingError::LengthOverflow)?,
    )?;
    length(
        module
            .functions
            .len()
            .checked_add(1)
            .ok_or(EncodingError::LengthOverflow)?,
    )?;
    length(module.data.len())?;
    if module.memory.pages > 65_536 {
        return Err(EncodingError::InvalidMemory);
    }
    length(module.memory.export.len())?;
    charge(&mut minimum, module.memory.export.len(), limit)?;
    for import in module.imports {
        length(import.module.len())?;
        length(import.name.len())?;
        charge(&mut minimum, import.module.len(), limit)?;
        charge(&mut minimum, import.name.len(), limit)?;
        charge(
            &mut minimum,
            usize::try_from(import.parameters).map_err(|_| EncodingError::LengthOverflow)?,
            limit,
        )?;
    }
    for (index, function) in module.functions.iter().enumerate() {
        length(function.name.len())?;
        charge(&mut minimum, function.name.len(), limit)?;
        charge(&mut minimum, function.calls.len(), limit)?;
        for (position, call) in function.calls.iter().enumerate() {
            let import = usize::try_from(call.import)
                .ok()
                .and_then(|index| module.imports.get(index));
            if import.is_none_or(|import| {
                Some(import.parameters) != u32::try_from(call.arguments.len()).ok()
            }) {
                return Err(EncodingError::InvalidCall {
                    function: index,
                    call: position,
                });
            }
            charge(&mut minimum, call.arguments.len(), limit)?;
        }
    }
    for (index, segment) in module.data.iter().enumerate() {
        let bytes = length(segment.bytes.len())?;
        if u64::from(segment.offset) + u64::from(bytes) > u64::from(module.memory.pages) * 65_536 {
            return Err(EncodingError::DataOutOfBounds { segment: index });
        }
        charge(&mut minimum, segment.bytes.len(), limit)?;
    }
    let mut exports = BTreeSet::new();
    exports.insert(module.memory.export);
    for function in module.functions {
        if !exports.insert(function.name) {
            return Err(EncodingError::DuplicateExport);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
