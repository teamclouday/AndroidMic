using System;
using System.Threading;
using AndroidMic.Audio;

namespace AndroidMic.Streaming
{
    public class StreamManager
    {
        public enum ConnectionType
        {
            BLUETOOTH,
            WIFI
        }

        public readonly int MAX_WAIT_TIME = 1000;
        public readonly int MIN_WAIT_TIME = 50;
        private volatile ConnectionType type = ConnectionType.BLUETOOTH;
        private readonly AudioBuffer sharedBuffer;
        private Streamer server;
        private Thread processThread;
        private volatile bool processAllowed;

        public event EventHandler<MessageArgs> AddLogEvent;
        public event EventHandler ServerListeningEvent;
        public event EventHandler ClientConnectedEvent;
        public event EventHandler ClientDisconnectedEvent;

        public StreamManager(AudioBuffer buffer)
        {
            sharedBuffer = buffer;
        }

        // start streaming server
        public void Start()
        {
            // skip if already started
            if(processThread != null && processThread.IsAlive)
            {
                AddLog("Server already started");
                return;
            }
            // start server
            try
            {
                switch(type)
                {
                    case ConnectionType.BLUETOOTH:
                        server = new StreamerBluetooth();
                        break;
                    case ConnectionType.WIFI:
                        server = new StreamerWifi();
                        break;
                }
            } catch(ArgumentException e)
            {
                server = null;
                AddLog("Error: " + e.Message);
                return;
            }
            if (processThread != null && processThread.IsAlive)
            {
                processAllowed = false;
                processThread.Join(MAX_WAIT_TIME);
            }
            processAllowed = true;
            processThread = new Thread(new ThreadStart(Process));
            processThread.Start();
            ServerListeningEvent?.Invoke(this, EventArgs.Empty);
            AddLog("Server Starts Listening...\n" + server.GetServerInfo());
        }

        // shutdown server
        public void Stop()
        {
            processAllowed = false;
            Thread.Sleep(MAX_WAIT_TIME);
            server?.Shutdown();
            server = null;
            if (processThread != null && processThread.IsAlive)
                processThread.Join(MAX_WAIT_TIME);
        }

        // process received data on thread
        public void Process()
        {
            // first wait for client connection
            while (processAllowed)
            {
                if (server == null)
                {
                    processAllowed = false;
                    break;
                }
                else if (server.IsAlive())
                {
                    ClientConnectedEvent?.Invoke(this, EventArgs.Empty);
                    AddLog("Client Connected\n" + server.GetClientInfo());
                    break;
                }
                Thread.Sleep(MIN_WAIT_TIME);
            }
            // then start process data
            while (processAllowed)
            {
                if (server == null || !server.IsAlive())
                {
                    processAllowed = false;
                    break;
                }
                server.Process(sharedBuffer);
            }
            ClientDisconnectedEvent?.Invoke(this, EventArgs.Empty);
            AddLog("Client Disconnected");
        }

        // check if stream server is running
        public bool IsRunning()
        {
            return processAllowed && server != null && server.IsAlive();
        }

        // set connection type
        // if server already running, cannot change
        public bool SetConnectionType(ConnectionType type)
        {
            if (server != null && server.IsAlive()) return false;
            this.type = type;
            return true;
        }

        // add log message to main window
        private void AddLog(string message)
        {
            AddLogEvent?.Invoke(this, new MessageArgs
            {
                Message = "[Connection Manager]\n" + message + "\n"
            });
        }
    }
}
