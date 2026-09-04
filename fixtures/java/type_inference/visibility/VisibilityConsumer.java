package app;

public class VisibilityConsumer {
    public String consumePublic(Visibility v) {
        return v.getPublic();
    }

    public String consumeDescribe(Visibility v) {
        return v.describe();
    }
}
