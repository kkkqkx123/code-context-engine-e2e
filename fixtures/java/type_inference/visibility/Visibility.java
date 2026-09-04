package app;

public class Visibility {
    public String pubField = "pub";
    protected String protectedField = "protected";
    String packageField = "package";
    private String privateField = "private";

    public String getPublic() {
        return pubField;
    }

    protected String getProtected() {
        return protectedField;
    }

    String getPackage() {
        return packageField;
    }

    private String getPrivate() {
        return privateField;
    }

    public String describe() {
        return getPublic() + getProtected() + getPackage() + getPrivate();
    }
}
