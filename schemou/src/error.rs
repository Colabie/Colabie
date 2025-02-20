macro_rules! error {
    [ $vis:vis enum $name:ident { $($variant:ident => $error_msg:expr),* $(,)? }] => {
        $vis enum $name {
            $($variant),*
        }

        impl SerdeError {
            fn description(&self) -> &'static str {
                match self {
                    $( Self::$variant => $error_msg, )*
                }
            }
        }

        impl std::error::Error for SerdeError {}
        impl std::fmt::Display for SerdeError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.description())
            }
        }

        impl std::fmt::Debug for SerdeError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match self {
                    $( Self::$variant => concat!("[", stringify!($variant), "] ", $error_msg), )*
                })
            }
        }
    };
}

error! {
    pub enum SerdeError {
        NotEnoughData => "ran out of data bytes while parsing, cannot deserialize the remaining fields",
        InvalidUTF8   => "raw bytes contain invalid UTF-8 data, cannot deserialize string",
        InvalidChars  => "found invalid characters for `ShortIdStr`",
    }
}
