#![cfg_attr(not(test), no_std)]

// trap crate example

pub fn hello() {
    // dummy function
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hello() {
        super::hello();
        assert_eq!(2 + 2, 4);
    }
}
