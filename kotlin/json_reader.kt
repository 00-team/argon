
interface FromJson<T> {
    abstract fun from_json(json_reader: JsonReader): T?
}

class JsonParseException(message: String) : Exception(message)

class JsonReader(private val json: String) {
    private var pos = 0

    // private val scope_stack = mutableListOf<Boolean>() // true = expecting comma before next item
    // private var current_scope_expects_comma = false

    fun debug(): String {
        return json.substring(pos)
    }

    // ---------- Low‑level helpers ----------
    private fun skip_whitespace() {
        while (pos < json.length && json[pos].isWhitespace()) pos++
    }

    private fun require_char(expected: Char) {
        if (json[pos] != expected) {
            throw JsonParseException("Expected '$expected' at position $pos, got '${json[pos]}'")
        }
        pos++
    }

    private fun peek(): Char = json[pos]

    // private fun consume_comma_if_needed() {
    //     skip_whitespace()
    //     if (current_scope_expects_comma) {
    //         if (peek() == ',') {
    //             pos++
    //             skip_whitespace()
    //         } else {
    //             throw JsonParseException("Expected ',' at position $pos, got '${peek()}'")
    //         }
    //     }
    // }
    //
    // private fun push_scope() {
    //     scope_stack.add(current_scope_expects_comma)
    //     current_scope_expects_comma = false // new scope starts without needing comma
    // }
    //
    // private fun pop_scope() {
    //     if (scope_stack.isEmpty()) {
    //         throw JsonParseException("Scope stack underflow")
    //     }
    //     current_scope_expects_comma = scope_stack.removeAt(scope_stack.lastIndex)
    // }

    // ---------- String parsing with escapes ----------
    private fun parse_string(): String {
        require_char('"')
        val sb = StringBuilder()
        while (pos < json.length) {
            when (val c = json[pos]) {
                '"' -> {
                    pos++
                    return sb.toString()
                }
                '\\' -> {
                    pos++
                    if (pos >= json.length) throw JsonParseException("Unterminated escape sequence")
                    val escaped = when (json[pos]) {
                        '"'  -> '"'
                        '\\' -> '\\'
                        '/'  -> '/'
                        'b'  -> '\b'
                        'f'  -> '\u000C'
                        'n'  -> '\n'
                        'r'  -> '\r'
                        't'  -> '\t'
                        'u'  -> {
                            if (pos + 4 >= json.length) throw JsonParseException("Incomplete unicode escape")
                            val hex = json.substring(pos + 1, pos + 5)
                            pos += 4
                            hex.toInt(16).toChar()
                        }
                        else -> throw JsonParseException("Invalid escape \\${json[pos]}")
                    }
                    sb.append(escaped)
                    pos++
                }
                else -> {
                    sb.append(c)
                    pos++
                }
            }
        }
        throw JsonParseException("Unterminated string")
    }

    // ---------- Number parsing ----------
    private fun parse_number(): Number {
        val start = pos
        while (pos < json.length && (json[pos].isDigit() || json[pos] == '.' || json[pos] == 'e' || json[pos] == 'E' || json[pos] == '+' || json[pos] == '-')) {
            pos++
        }
        val numStr = json.substring(start, pos)
        return if ('.' in numStr || 'e' in numStr || 'E' in numStr) {
            numStr.toDouble()
        } else {
            numStr.toInt()
        }
    }

    // ---------- Literals: true, false, null ----------
    private fun parse_literal(expected: String): Boolean {
        val end = pos + expected.length
        if (end > json.length) throw JsonParseException("Unexpected end of input")
        val actual = json.substring(pos, end)
        if (actual != expected) throw JsonParseException("Expected '$expected' at $pos, got '$actual'")
        pos = end
        return true
    }

    // ---------- Value skipping ----------
    fun skip_value() {
        skip_whitespace()
        when (val c = peek()) {
            '"' -> { parse_string() }
            '{' -> {
                begin_obj()
                while (has_next()) {
                    next_name()
                    skip_value()
                }
                end_obj()
            }
            '[' -> {
                begin_array()
                while (has_next()) {
                    skip_value()
                }
                end_array()
            }
            't', 'f' -> { parse_literal(if (c == 't') "true" else "false") }
            'n' -> { parse_literal("null") }
            else -> {
                if (c.isDigit() || c == '-') {
                    parse_number()
                } else {
                    throw JsonParseException("Unexpected character '$c' at $pos")
                }
            }
        }
    }

    // ---------- Public typed readers ----------
    fun next_string(): String {
        skip_whitespace()
        return parse_string()
    }

    fun next_int(): Int {
        skip_whitespace()
        return parse_number().toInt()
    }

    fun next_double(): Double {
        skip_whitespace()
        return parse_number().toDouble()
    }

    fun next_bool(): Boolean {
        skip_whitespace()
        return when (peek()) {
            't' -> { parse_literal("true"); true }
            'f' -> { parse_literal("false"); false }
            else -> throw JsonParseException("Expected boolean at $pos")
        }
    }

    fun next_null(): Unit? {
        skip_whitespace()
        parse_literal("null")
        return null
    }

    // ---------- Object handling ----------
    fun begin_obj() {
        skip_whitespace()
        require_char('{')
        // push_scope()
    }

    fun end_obj() {
        skip_whitespace()
        require_char('}')
        // pop_scope()
    }

    fun has_next(): Boolean {
        skip_whitespace()

        if (pos >= json.length) return false
        if (peek() == ',') {
            pos++
            skip_whitespace()
        }
        if (pos >= json.length) return false
        
        val c = peek()
        if (c == '}' || c == ']') return false

        // current_scope_expects_comma = true
        return true
    }

    fun next_name(): String {
        skip_whitespace()
        // If not the first key, expect a comma
        // if (json[pos] == ',') {
        //     pos++
        //     skip_whitespace()
        // }
        val key = parse_string()
        skip_whitespace()
        require_char(':')
        return key
    }

    fun peek_tag(key: String): String? {
        val old_pos = pos
        begin_obj()
        while (has_next()) {
            if (next_name() == key) {
                val tag = next_string()
                pos = old_pos
                return tag
            }
            skip_value()
        }
        end_obj()
        pos = old_pos
        return null
    }

    // ---------- Array handling ----------
    fun begin_array() {
        skip_whitespace()
        require_char('[')
        // push_scope()
    }

    fun end_array() {
        skip_whitespace()
        require_char(']')
        // pop_scope()
    }

    // null handling
    fun is_next_null(): Boolean {
        skip_whitespace()
        return peek() == 'n'
    }

    fun next_int_or_null(): Int? {
        return if (is_next_null()) {
            next_null()
            null
        } else {
            next_int()
        }
    }

    fun next_double_or_null(): Double? {
        return if (is_next_null()) {
            next_null()
            null
        } else {
            next_double()
        }
    }

    fun next_string_or_null(): String? {
        return if (is_next_null()) {
            next_null()
            null
        } else {
            next_string()
        }
    }

    fun next_bool_or_null(): Boolean? {
        return if (is_next_null()) {
            next_null()
            null
        } else {
            next_bool()
        }
    }
}
