using NAudio.CoreAudioApi;
using NAudio.Wave;
using System.Windows.Forms;
using System.Threading;
using System;

namespace Experimental
{
    partial class Program
    {
        public static void TestWasapiOut()
        {
            var dialog = new OpenFileDialog
            {
                Multiselect = false,
                Title = "Select Audio",
                Filter = "Mp3|*.mp3|WAV|*.wav|All|*.*",
                RestoreDirectory = true,
            };
            if (dialog.ShowDialog() == DialogResult.OK)
            {
                var path = dialog.FileName;
                var reader = new AudioFileReader(path);
                var deviceIter = new MMDeviceEnumerator();
                var player = new WasapiOut(deviceIter.GetDefaultAudioEndpoint(DataFlow.Render, Role.Console), AudioClientShareMode.Shared, false, 100);
                player.Init(reader);
                player.Play();
                Console.WriteLine("Press space to stop player");
                while (player.PlaybackState == PlaybackState.Playing)
                {
                    if (Console.KeyAvailable)
                    {
                        if (Console.ReadKey().Key == ConsoleKey.Spacebar)
                        {
                            Console.WriteLine("Stopping player...");
                            player.Stop();
                            Console.WriteLine("Player stopped");
                            break;
                        }
                    }
                }
                player.Stop();
                player.Dispose();
            }
        }
    }
}
