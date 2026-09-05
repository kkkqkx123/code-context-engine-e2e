package app.models

class Visibility {
  val pubField: String = "public"
  private val privateField: String = "private"
  protected val protectedField: String = "protected"

  def getPublic(): String = pubField

  private def getPrivate(): String = privateField

  protected def getProtected(): String = protectedField

  def describe(): String = getPublic() + getPrivate() + getProtected()
}
