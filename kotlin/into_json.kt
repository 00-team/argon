
interface IntoJson {
    fun into_json(): String
}

private fun json_escape(str: String): String = str.replace("\\", "\\\\").replace("\"", "\\\"")


// The one and only function
fun into_json(value: Any?): String = when (value) {
    null -> "null"
    is String -> "\"${json_escape(value)}\""
    is Number, is Boolean -> value.toString()
    is List<*> -> value.joinToString(prefix = "[", postfix = "]") { into_json(it) }
    is Map<*, *> -> value.entries.joinToString(prefix = "{", postfix = "}") { (k, v) ->
        "\"$k\":${into_json(v)}"
    }
    is IntoJson -> value.into_json()
    else -> error("Don't know how to convert ${value::class.simpleName} to JSON")
}

// Optional: extension method for dot syntax
// fun Any?.into_json() = into_json(this)
