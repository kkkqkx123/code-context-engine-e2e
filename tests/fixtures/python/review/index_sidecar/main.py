def process_values(values):
    if values is not None:
        buffer = []
        for item in values:
            if item < 0:
                continue
            buffer.append(item)

        borrowed = len(buffer)
        _ = borrowed
        shifted = 1 << 2

        while True:
            if len(buffer) == 0:
                outcome = len(buffer)
                break
            else:
                outcome = len(buffer)
                break

        return outcome
    else:
        raise ValueError("missing values")
