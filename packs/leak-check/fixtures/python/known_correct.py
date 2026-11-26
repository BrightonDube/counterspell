def leak_check(train, test):
    return len(set(train) & set(test))
