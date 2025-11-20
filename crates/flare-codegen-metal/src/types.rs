use crate::error::{CodegenError, Result};
use flare_ir::hir::*;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetalType {
    pub msl_type: String,

    pub size_bytes: Option<usize>,

    pub alignment: Option<usize>,
}

impl MetalType {
    pub fn new(msl_type: impl Into<String>) -> Self {
        Self {
            msl_type: msl_type.into(),
            size_bytes: None,
            alignment: None,
        }
    }

    pub fn with_layout(msl_type: impl Into<String>, size_bytes: usize, alignment: usize) -> Self {
        Self {
            msl_type: msl_type.into(),
            size_bytes: Some(size_bytes),
            alignment: Some(alignment),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.msl_type
    }
}

pub struct TypeConverter;

impl TypeConverter {
    pub fn convert(ty: &Type, span: Range<usize>) -> Result<MetalType> {
        match ty {
            Type::I32 => Ok(MetalType::with_layout("int", 4, 4)),
            Type::I64 => Ok(MetalType::with_layout("long", 8, 8)),
            Type::U32 => Ok(MetalType::with_layout("uint", 4, 4)),
            Type::U64 => Ok(MetalType::with_layout("ulong", 8, 8)),
            Type::F32 => Ok(MetalType::with_layout("float", 4, 4)),
            Type::F64 => Ok(MetalType::with_layout("double", 8, 8)),
            Type::Bool => Ok(MetalType::with_layout("bool", 1, 1)),

            Type::Vector { dtype, len } => Self::convert_vector(dtype, len.as_ref(), span),

            Type::Matrix { dtype, rows, cols } => {
                Self::convert_matrix(dtype, rows.as_ref(), cols.as_ref(), span)
            }

            Type::Ptr(inner) => {
                let inner_type = Self::convert(inner, span.clone())?;

                Ok(MetalType::new(format!("device {}*", inner_type.as_str())))
            }

            Type::Array { dtype, size } => {
                let elem_type = Self::convert(dtype, span.clone())?;
                match size {
                    Some(n) => Ok(MetalType::new(format!("{}[{}]", elem_type.as_str(), n))),
                    None => Ok(MetalType::new(format!("device {}*", elem_type.as_str()))),
                }
            }

            Type::Tensor { dtype, .. } => {
                let elem_type = Self::convert(dtype, span.clone())?;
                Ok(MetalType::new(format!("device {}*", elem_type.as_str())))
            }

            Type::Named(name) => {
                if Self::is_known_metal_type(name) {
                    Ok(MetalType::new(*name))
                } else {
                    Err(CodegenError::unsupported_type(
                        format!("unknown type '{}'", name),
                        span,
                    ))
                }
            }
        }
    }

    fn convert_vector(dtype: &Type, len: Option<&&str>, span: Range<usize>) -> Result<MetalType> {
        let base_type = Self::convert(dtype, span.clone())?;

        let length = match len {
            Some(&"2") | Some(&"x") => "2",
            Some(&"3") | Some(&"y") => "3",
            Some(&"4") | Some(&"z") => "4",
            Some(other) => match other.parse::<usize>() {
                Ok(2) => "2",
                Ok(3) => "3",
                Ok(4) => "4",
                _ => {
                    return Err(CodegenError::unsupported_type(
                        format!(
                            "Metal only supports vector lengths 2, 3, 4, got '{}'",
                            other
                        ),
                        span,
                    ));
                }
            },
            None => {
                return Err(CodegenError::unsupported_type(
                    "vector type requires explicit length in Metal",
                    span,
                ));
            }
        };

        let type_prefix = match base_type.as_str() {
            "int" => "int",
            "uint" => "uint",
            "float" => "float",
            "double" => "double",
            "bool" => "bool",
            "short" => "short",
            "ushort" => "ushort",
            "char" => "char",
            "uchar" => "uchar",
            _ => {
                return Err(CodegenError::unsupported_type(
                    format!("cannot create vector of type '{}'", base_type.as_str()),
                    span,
                ));
            }
        };

        Ok(MetalType::new(format!("{}{}", type_prefix, length)))
    }

    fn convert_matrix(
        dtype: &Type,
        rows: Option<&&str>,
        cols: Option<&&str>,
        span: Range<usize>,
    ) -> Result<MetalType> {
        let base_type = Self::convert(dtype, span.clone())?;

        match base_type.as_str() {
            "float" | "half" => {}
            other => {
                return Err(CodegenError::unsupported_type(
                    format!("Metal matrices only support float/half, got '{}'", other),
                    span,
                ));
            }
        }

        let rows_num = Self::parse_dimension(rows, "rows", &span)?;
        let cols_num = Self::parse_dimension(cols, "cols", &span)?;

        if !(2..=4).contains(&rows_num) || !(2..=4).contains(&cols_num) {
            return Err(CodegenError::unsupported_type(
                format!(
                    "Metal matrices must be 2x2 to 4x4, got {}x{}",
                    rows_num, cols_num
                ),
                span,
            ));
        }

        Ok(MetalType::new(format!(
            "{}{}x{}",
            base_type.as_str(),
            cols_num,
            rows_num
        )))
    }

    fn parse_dimension(dim: Option<&&str>, name: &str, span: &Range<usize>) -> Result<usize> {
        match dim {
            Some(s) => s.parse::<usize>().map_err(|_| {
                CodegenError::unsupported_type(
                    format!("invalid matrix dimension for {}: '{}'", name, s),
                    span.clone(),
                )
            }),
            None => Err(CodegenError::unsupported_type(
                format!("matrix {} dimension required in Metal", name),
                span.clone(),
            )),
        }
    }

    fn is_known_metal_type(name: &str) -> bool {
        matches!(
            name,
            "int"
                | "uint"
                | "float"
                | "double"
                | "bool"
                | "short"
                | "ushort"
                | "char"
                | "uchar"
                | "long"
                | "ulong"
                | "half"
                | "size_t"
                | "ptrdiff_t"
                | "int2"
                | "int3"
                | "int4"
                | "uint2"
                | "uint3"
                | "uint4"
                | "float2"
                | "float3"
                | "float4"
                | "half2"
                | "half3"
                | "half4"
                | "float2x2"
                | "float3x3"
                | "float4x4"
                | "half2x2"
                | "half3x3"
                | "half4x4"
        )
    }

    pub fn address_space_for_location(location: &str) -> &'static str {
        match location {
            "shared" | "threadgroup" => "threadgroup",
            "constant" | "const" => "constant",
            "device" | "global" => "device",
            _ => "device",
        }
    }
}
