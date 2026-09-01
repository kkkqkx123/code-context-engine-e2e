package example

class DataProcessor {
    private val items = mutableListOf<String>()

    fun addItem(item: String) {
        items.add(item)
    }

    fun getItems(): List<String> = items.toList()

    fun count(): Int = items.size
}
