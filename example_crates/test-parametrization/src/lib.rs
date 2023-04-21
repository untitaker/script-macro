pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[script_macro::run_script_on(r##"
        let output = item;

        for x in 0..10 {
            for y in 0..10 {
                output += `
                #[test]
                fn it_works_${x}_${y}() {
                    it_works(${x}, ${y}, ${x + y});
                }`;
            }
        }

        return output;
    "##)]
    fn it_works(x: usize, y: usize, out: usize) {
        assert_eq!(add(x, y), out);
    }
}
