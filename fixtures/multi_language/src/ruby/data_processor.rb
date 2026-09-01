class DataProcessor
  def initialize
    @items = []
  end

  def add_item(item)
    @items << item
  end

  def get_items
    @items.dup
  end

  def count
    @items.size
  end
end
