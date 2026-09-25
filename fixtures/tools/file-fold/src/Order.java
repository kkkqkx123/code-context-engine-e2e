package com.example.demo;

import java.util.ArrayList;
import java.util.List;

public class Order {
    private final long id;
    private String status;

    public Order(long id) {
        this.id = id;
        this.status = "open";
    }

    public long getId() {
        return id;
    }

    public String getStatus() {
        return status;
    }

    public void cancel() {
        if (!"closed".equals(status)) {
            status = "cancelled";
        }
    }

    public void close() {
        status = "closed";
    }
}

interface OrderRepository {
    Order find(long id);

    void save(Order order);
}

class InMemoryOrderRepository implements OrderRepository {
    private final List<Order> orders = new ArrayList<>();

    @Override
    public Order find(long id) {
        for (Order order : orders) {
            if (order.getId() == id) {
                return order;
            }
        }
        return null;
    }

    @Override
    public void save(Order order) {
        orders.add(order);
    }
}
