mod file;


#[cfg(test)]
mod tests {
    use crate::file;

    #[test]
    fn it_works() {
       let v =  file::test();
        let result = 2 + 2;
        let a = 2;
        assert_eq!(result, 4);
    }
}
