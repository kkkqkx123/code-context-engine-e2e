package app.models

open class Visibility {
    val pubField: String = "public"
    internal val internalField: String = "internal"
    protected val protectedField: String = "protected"
    private val privateField: String = "private"

    fun getPublic(): String = pubField

    internal fun getInternal(): String = internalField

    protected fun getProtected(): String = protectedField

    private fun getPrivate(): String = privateField

    fun describe(): String = getPublic() + getInternal() + getProtected() + getPrivate()
}
