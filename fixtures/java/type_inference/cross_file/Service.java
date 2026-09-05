package app.service;

import app.models.User;
import app.models.UserFactory;

public class Service {
    public String renderGreeting(User user) {
        return user.greet();
    }

    public void main() {
        User user = UserFactory.createUser("Alice");
        System.out.println(renderGreeting(user));
        System.out.println(user.greet());
    }
}
