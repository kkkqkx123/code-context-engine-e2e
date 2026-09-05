export class Visibility {
    public pubField: string = "public";
    protected protectedField: string = "protected";
    private privateField: string = "private";
    readonly readonlyField: string = "readonly";

    public getPublic(): string {
        return this.pubField;
    }

    protected getProtected(): string {
        return this.protectedField;
    }

    private getPrivate(): string {
        return this.privateField;
    }

    public describe(): string {
        return this.getPublic() + this.getProtected() + this.getPrivate();
    }
}
