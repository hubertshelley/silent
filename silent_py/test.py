import signal
from silent_py import SilentServer

server = SilentServer()

signal.signal(signal.SIGINT, server.stop)
server.set_logger()
server.run()
