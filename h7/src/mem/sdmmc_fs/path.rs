#[derive(Debug)]
pub struct Path<'p> {
    // raw: &'p str,
    path: &'p str,
    device: Option<&'p str>,
    absolute: bool,
}

impl<'p> Path<'p> {
    pub fn new<P: AsRef<str> + ?Sized>(raw: &'p P) -> Self {
        let raw_trimmed = raw.as_ref().trim();
        let (raw_path, device) = match raw_trimmed.find(':') {
            Some(n) => (raw_trimmed[n + 1..].trim(), Some(raw_trimmed[..n].trim())),
            None => (raw_trimmed.trim(), None),
        };
        let path = match raw_path {
            "" | "/" => raw_path,
            _ => {
                match raw_path.find(|c| c != '/' && !char::is_whitespace(c)) {
                    Some(n) => raw_path[n.saturating_sub(1)..]
                        .trim_end_matches(|c| c == '/' || char::is_whitespace(c)),
                    // raw_path is all slashes/space, emtpy case is already handled
                    None => raw_path[0..1].trim(),
                }
            }
        };
        Self {
            // raw: raw_trimmed,
            path,
            device,
            absolute: path.starts_with('/') || device.is_some(),
        }
    }

    // pub fn raw(&self) -> &'p str {
    //     self.raw
    // }

    // pub fn path(&self) -> &'p str {
    //     self.path
    // }

    // pub fn device(&self) -> Option<&'p str> {
    //     self.device
    // }

    // pub fn is_absolute(&self) -> bool {
    //     self.absolute
    // }

    pub fn parts(&self) -> core::iter::Peekable<impl Iterator<Item = &'p str>> {
        self.path
            .split('/')
            .filter_map(|p| (!p.trim().is_empty()).then(|| p.trim()))
            .peekable()
    }

    // pub fn len(&self) -> usize {
    //     self.parts().count()
    // }
}

impl<'p> core::fmt::Display for Path<'p> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(device) = self.device {
            write!(f, "{device}:")?;
        }
        if self.absolute {
            write!(f, "/")?;
        }
        let mut pi = self.parts();
        let mut next = pi.next();
        while next.is_some() {
            if let Some(n) = next {
                write!(f, "{n}")?;
            }
            next = pi.next();
            if next.is_some() {
                write!(f, "/")?;
            }
        }
        Ok(())
    }
}

impl<'s, T: AsRef<str> + ?Sized + 's> From<&'s T> for Path<'s> {
    fn from(p: &'s T) -> Self {
        Self::new(p.as_ref())
    }
}

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn absolute_path() {
//         let p = Path::new("/abs/path/hello");
//         assert_eq!(p.raw(), "/abs/path/hello");
//         assert_eq!(p.path(), "/abs/path/hello");
//         assert_eq!(p.device(), None);
//         assert!(p.is_absolute());
//         assert_eq!(p.to_string(), "/abs/path/hello");
//         assert_eq!(p.len(), 3);
//         let mut parts = p.parts();
//         assert_eq!(parts.next(), Some("abs"));
//         assert_eq!(parts.next(), Some("path"));
//         assert_eq!(parts.next(), Some("hello"));
//         assert_eq!(parts.next(), None);
//     }

//     #[test]
//     fn relative_path() {
//         let p = Path::new("  r/path/hello  ");
//         assert_eq!(p.raw(), "r/path/hello");
//         assert_eq!(p.path(), "r/path/hello");
//         assert_eq!(p.device(), None);
//         assert!(!p.is_absolute());
//         assert_eq!(p.to_string(), "r/path/hello");
//         assert_eq!(p.len(), 3);
//         let mut parts = p.parts();
//         assert_eq!(parts.next(), Some("r"));
//         assert_eq!(parts.next(), Some("path"));
//         assert_eq!(parts.next(), Some("hello"));
//         assert_eq!(parts.next(), None);
//     }

//     #[test]
//     fn absolute_path_with_device() {
//         let p = Path::new("A:  /abs/path/hello ");
//         assert_eq!(p.raw(), "A:  /abs/path/hello");
//         assert_eq!(p.path(), "/abs/path/hello");
//         assert_eq!(p.device(), Some("A"));
//         assert!(p.is_absolute());
//         assert_eq!(p.to_string(), "A:/abs/path/hello");
//         assert_eq!(p.len(), 3);
//         let mut parts = p.parts();
//         assert_eq!(parts.next(), Some("abs"));
//         assert_eq!(parts.next(), Some("path"));
//         assert_eq!(parts.next(), Some("hello"));
//         assert_eq!(parts.next(), None);
//     }

//     #[test]
//     fn absolute_path_with_device_many_space() {
//         let p = Path::new("  A:      / / / /abs/path/hello/ / /  ");
//         assert_eq!(p.raw(), "A:      / / / /abs/path/hello/ / /");
//         assert_eq!(p.path(), "/abs/path/hello");
//         assert_eq!(p.device(), Some("A"));
//         assert!(p.is_absolute());
//         assert_eq!(p.to_string(), "A:/abs/path/hello");
//         assert_eq!(p.len(), 3);
//         let mut parts = p.parts();
//         assert_eq!(parts.next(), Some("abs"));
//         assert_eq!(parts.next(), Some("path"));
//         assert_eq!(parts.next(), Some("hello"));
//         assert_eq!(parts.next(), None);
//     }

//     #[test]
//     fn relative_path_with_device() {
//         let p = Path::new("A:rel/path/hello");
//         assert_eq!(p.raw(), "A:rel/path/hello");
//         assert_eq!(p.path(), "rel/path/hello");
//         assert_eq!(p.device(), Some("A"));
//         assert!(p.is_absolute());
//         assert_eq!(p.to_string(), "A:/rel/path/hello");
//         assert_eq!(p.len(), 3);
//         let mut parts = p.parts();
//         assert_eq!(parts.next(), Some("rel"));
//         assert_eq!(parts.next(), Some("path"));
//         assert_eq!(parts.next(), Some("hello"));
//         assert_eq!(parts.next(), None);
//     }

//     #[test]
//     fn absolute_root() {
//         let p = Path::new("/");
//         assert_eq!(p.raw(), "/");
//         assert_eq!(p.path(), "/");
//         assert_eq!(p.device(), None);
//         assert!(p.is_absolute());
//         assert_eq!(p.to_string(), "/");
//         assert_eq!(p.len(), 0);
//         let mut parts = p.parts();
//         assert_eq!(parts.next(), None);
//     }

//     #[test]
//     fn absolute_root_with_device() {
//         let p = Path::new("A:/");
//         assert_eq!(p.raw(), "A:/");
//         assert_eq!(p.path(), "/");
//         assert_eq!(p.device(), Some("A"));
//         assert!(p.is_absolute());
//         assert_eq!(p.to_string(), "A:/");
//         assert_eq!(p.len(), 0);
//         let mut parts = p.parts();
//         assert_eq!(parts.next(), None);
//     }

//     #[test]
//     fn empty() {
//         let p = Path::new("");
//         assert_eq!(p.raw(), "");
//         assert_eq!(p.path(), "");
//         assert_eq!(p.device(), None);
//         assert!(!p.is_absolute());
//         assert_eq!(p.to_string(), "");
//         assert_eq!(p.len(), 0);
//         let mut parts = p.parts();
//         assert_eq!(parts.next(), None);
//     }

//     #[test]
//     fn empty_with_device() {
//         let p = Path::new("A:");
//         assert_eq!(p.raw(), "A:");
//         assert_eq!(p.path(), "");
//         assert_eq!(p.device(), Some("A"));
//         assert!(p.is_absolute());
//         assert_eq!(p.to_string(), "A:/");
//         assert_eq!(p.len(), 0);
//         let mut parts = p.parts();
//         assert_eq!(parts.next(), None);
//     }

//     #[test]
//     fn absolute_path_many_with_device() {
//         let p = Path::new("A:////abs/path/hello///");
//         assert_eq!(p.raw(), "A:////abs/path/hello///");
//         assert_eq!(p.path(), "/abs/path/hello");
//         assert_eq!(p.device(), Some("A"));
//         assert!(p.is_absolute());
//         assert_eq!(p.to_string(), "A:/abs/path/hello");
//         assert_eq!(p.len(), 3);
//         let mut parts = p.parts();
//         assert_eq!(parts.next(), Some("abs"));
//         assert_eq!(parts.next(), Some("path"));
//         assert_eq!(parts.next(), Some("hello"));
//         assert_eq!(parts.next(), None);
//     }

//     #[test]
//     fn absolute_path_many() {
//         let p = Path::new("////abs/path/hello///");
//         assert_eq!(p.raw(), "////abs/path/hello///");
//         assert_eq!(p.path(), "/abs/path/hello");
//         assert_eq!(p.device(), None);
//         assert!(p.is_absolute());
//         assert_eq!(p.to_string(), "/abs/path/hello");
//         assert_eq!(p.len(), 3);
//         let mut parts = p.parts();
//         assert_eq!(parts.next(), Some("abs"));
//         assert_eq!(parts.next(), Some("path"));
//         assert_eq!(parts.next(), Some("hello"));
//         assert_eq!(parts.next(), None);
//     }

//     #[test]
//     fn absolute_root_many() {
//         let p = Path::new("///////////////////");
//         assert_eq!(p.raw(), "///////////////////");
//         assert_eq!(p.path(), "/");
//         assert_eq!(p.device(), None);
//         assert!(p.is_absolute());
//         assert_eq!(p.to_string(), "/");
//         assert_eq!(p.len(), 0);
//         let mut parts = p.parts();
//         assert_eq!(parts.next(), None);
//     }

//     #[test]
//     fn absolute_root_many_with_device() {
//         let p = Path::new("A://////////////////");
//         assert_eq!(p.raw(), "A://////////////////");
//         assert_eq!(p.path(), "/");
//         assert_eq!(p.device(), Some("A"));
//         assert!(p.is_absolute());
//         assert_eq!(p.to_string(), "A:/");
//         assert_eq!(p.len(), 0);
//         let mut parts = p.parts();
//         assert_eq!(parts.next(), None);
//     }

//     #[test]
//     fn new_with_string() {
//         let s = String::from("A:");
//         let p = Path::new(&s);
//         assert!(p.is_absolute());
//     }

//     #[test]
//     fn new_with_str() {
//         let s = "A:";
//         let p = Path::new(s);
//         assert!(p.is_absolute());
//     }

//     #[test]
//     fn from_string() {
//         let s = String::from("A:");
//         let p = Path::from(&s);
//         assert!(p.is_absolute());
//     }

//     #[test]
//     fn from_str() {
//         let s = "A:";
//         let p = Path::from(s);
//         assert!(p.is_absolute());
//     }

//     #[test]
//     fn from_str_literal() {
//         let p = Path::from("A:");
//         assert!(p.is_absolute());
//     }
// }
