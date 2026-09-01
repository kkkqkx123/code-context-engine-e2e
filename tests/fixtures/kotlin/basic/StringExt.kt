package app

fun String.isPalindrome(): Boolean {
    val cleaned = this.lowercase().filter { it.isLetterOrDigit() }
    return cleaned == cleaned.reversed()
}

fun String.wordCount(): Int = this.trim().split("\\s+".toRegex()).size
