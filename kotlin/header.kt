// class Logger {
//     companion object {
//         fun debug(tag: String, msg: String) {}
//         fun verbose(tag: String, msg: String) {}
//     }
// }

fun assert_eq(a: Any?, b: Any?) {
    if (a != b) {
        throw AssertionError("unequal: $a != $b")
    }
}

