def main():
    result = process()
    print(f"Result: {result}")

def process():
    x = helper()
    return x * 2

def helper():
    return 21

if __name__ == "__main__":
    main()
