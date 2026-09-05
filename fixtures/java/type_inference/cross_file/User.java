package app.models;

public class User {
    private String name;
    private int age;

    public User(String name, int age) {
        this.name = name;
        this.age = age;
    }

    public String getName() {
        return name;
    }

    public int getAge() {
        return age;
    }

    public String greet() {
        return "Hello, " + name + "!";
    }
}

class UserFactory {
    public static User createUser(String name) {
        return new User(name, 30);
    }
}
