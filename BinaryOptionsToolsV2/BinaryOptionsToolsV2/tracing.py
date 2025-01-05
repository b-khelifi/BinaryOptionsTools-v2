from BinaryOptionsToolsV2 import start_tracing

def start_logs(path: str, level: str = "DEBUG", terminal: bool = True):
    """
    Initialize logging system for the application.

    Args:
        path (str): Path where log files will be stored.
        level (str): Logging level (default is "DEBUG").
        terminal (bool): Whether to display logs in the terminal (default is True).

    Returns:
        None

    Raises:
        Exception: If there's an error starting the logging system.
    """

    try:
        start_tracing(path, level, terminal)
    except Exception as e:
        print(f"Error starting logs, {e}")