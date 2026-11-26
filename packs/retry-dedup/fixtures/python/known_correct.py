def retry_dedup(log):
    return len({rite for _, rite in log})
