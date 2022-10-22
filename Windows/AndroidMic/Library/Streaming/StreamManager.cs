using System;
using System.Diagnostics;
using System.Threading;
using System.Threading.Tasks;
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
        private Task processTask;
        private volatile bool processAllowed;
        private CancellationTokenSource cancellationTokenSource;

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
            if (processTask?.IsCompleted == false)
            {
                AddLog("Server already started");
                return;
            }
            // start server
            try
            {
                switch (type)
                {
                    case ConnectionType.BLUETOOTH:
                        server = new StreamerBluetooth();
                        break;
                    case ConnectionType.WIFI:
                        server = new StreamerWifi();
                        break;
                }
            }
            catch (ArgumentException e)
            {
                server = null;
                AddLog("Error: " + e.Message);
                return;
            }
            processAllowed = true;
            cancellationTokenSource = new CancellationTokenSource();
            processTask = Task.Factory.StartNew(Process, cancellationTokenSource.Token, TaskCreationOptions.LongRunning, TaskScheduler.Default);
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
            if (processTask?.IsCompleted == false)
            {
                cancellationTokenSource.Cancel();
                try
                {
                    processTask?.Wait();
                }
                catch (OperationCanceledException err)
                {
                    Debug.WriteLine("[StreamManager] Stop -> " + err.Message);
                }
                finally
                {
                    processTask?.Dispose();
                }
            }
        }

        // process received data on thread
        public async void Process()
        {
            // first wait for client connection
            while (processAllowed)
            {
                if(cancellationTokenSource.IsCancellationRequested)
                {
                    break;
                }
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
                await Task.Delay(MIN_WAIT_TIME);
            }
            // then start process data
            while (processAllowed)
            {
                if (cancellationTokenSource.IsCancellationRequested)
                {
                    break;
                }
                if (server == null || !server.IsAlive())
                {
                    processAllowed = false;
                    break;
                }
                await server.Process(sharedBuffer);
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
