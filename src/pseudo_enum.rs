#[derive(std::fmt::Debug)]
pub struct ConversionError {
	pub r#type: String,
	pub value: String,
}

impl std::fmt::Display for ConversionError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "no such {}: {}", self.r#type, self.value)
	}
}

impl std::error::Error for ConversionError { }

macro_rules! pseudo_enum {
	($name: ident : $type: ty { $( $value: expr => $variant: ident ),* $(,)? }) => {
		#[derive(Copy, Clone, PartialEq, Eq, Hash, serde::Deserialize)]
		#[serde(transparent)]
		pub struct $name(pub $type);

		impl $name {
			$( pub const $variant:$name = $name($value); )*
		}

        impl From<$name> for $type {
			fn from(val: $name) -> $type {
				match val {
                    $( $name($value) => $value, )*
                    _ => 0,
				}
			}
        }

		impl std::convert::TryFrom<&str> for $name {
			type Error = super::pseudo_enum::ConversionError;
			fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
				match s {
					$( stringify!($variant) => Ok($name::$variant), )*
					_ => Err(Self::Error {
						r#type: format!("{}::{}", module_path!(), stringify!($name)),
						value: s.to_string(),
					}),
				}
			}
		}

		impl std::fmt::Debug for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match unsafe { super::SERIALIZATION_CONFIG.enum_names } {
					true => match self.0 {
						$( $value => write!(f, "{}:{}", self.0, stringify!($variant)), )*
						_ => write!(f, "{}", self.0),
					},
					_ => write!(f, "{}", self.0),
				}
			}
		}

		impl serde::Serialize for $name {
			fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
				match unsafe { super::SERIALIZATION_CONFIG.enum_names } {
					true => format!("{:?}", self).serialize(serializer),
					_ => self.0.serialize(serializer),
				}
			}
		}

		impl super::arrow::ArrowPrimitive for $name {
			type ArrowNativeType = $type;
			type ArrowType = <$type as super::arrow::ArrowPrimitive>::ArrowType;
			const ARROW_DATA_TYPE: ::arrow::datatypes::DataType = <$type>::ARROW_DATA_TYPE;
			fn into_arrow_native(self) -> Self::ArrowNativeType { self.0 as $type }
		}
	}
}
