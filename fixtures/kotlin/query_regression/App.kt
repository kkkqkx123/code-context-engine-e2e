import shop.cart.Item
import shop.cart.*
import shop.cart.Helper as CartHelper

class Store {
  fun open(): Item {
    val item = Item()
    cart.add(item)
    cart?.checkout()
    return cart.items().first()
  }
}

fun String.shout(): String = this.uppercase()
