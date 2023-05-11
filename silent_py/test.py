import signal
from silent_py import SilentServer

server = SilentServer()
server.set_logger()
server.run()
