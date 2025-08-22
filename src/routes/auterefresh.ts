let intervalId: number | null = null;

export function enableAutoRefresh(cb: Function): () => void {
  if (intervalId !== null) {
    clearInterval(intervalId);
  }
  const interval = setInterval(cb, 1000 * 60 * 30);
  intervalId = interval;

  return () => {
    clearInterval(interval);
  };
}

export function disabledAutoRefresh() {
  if (intervalId !== null) clearInterval(intervalId);
}
