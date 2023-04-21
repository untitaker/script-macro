pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[script_macro::run_script_on(r##"
        let output = item;

        for testcase in parse_json(open_file("testcases.json").read_string()) {
            let a = testcase[0];
            let b = testcase[1];
            let result = testcase[2];
            let fn_name = slugify_ident(`it_works_${a}_${b}`);
            output += `
            #[test]
            fn ${fn_name}() {
                it_works(${a}, ${b}, ${result});
            }
            `;
        }

        return output;
    "##)]
    fn it_works(a: usize, b: usize, output: usize) {
        assert_eq!(add(a, b), output);
    }
}
