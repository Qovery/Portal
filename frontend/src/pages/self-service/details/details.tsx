import { useEffect, useState } from "react";
import { API_URL } from "../../../config";
import { Link, useParams } from "react-router-dom";
import { XTerm } from 'react-xtermjs'
import { DataLoggerAddon } from "./xterm-addon";
import { ITerminalAddon } from "@xterm/xterm";
import { FitAddon } from '@xterm/addon-fit';
import { getTotalExecutionTime } from "../helpers/get-total-execution-time";
import { millisToHumanTime } from "../../../lib/utils";

export type Log = {
  id: string;
  createdAt: string;
  isStderr: boolean;
  message: string;
}

type SelfServiceRun = {
  id: string;
  section_slug: string;
  action_slug: string;
  status: "SUCCESS";
  tasks: Array<any>;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function mapLog(incomingLog: any): Log {
  return {
    ...incomingLog,
    createdAt: incomingLog.created_at,
    isStderr: incomingLog.is_stderr,
  }
}

export const Details = () => {
  const { taskId } = useParams();
  const fitAddon = new FitAddon()

  const [data, setData] = useState<SelfServiceRun | undefined>(undefined);
  const [addons, setAddons] = useState<Array<ITerminalAddon>>([]);
  const [executionTime, setExecutionTime] = useState<number>(0);

  useEffect(() => {
    fetch(`${API_URL}/selfServiceSections/${taskId}`)
      .then(res => res.json())
      .then((data) => {
        setData(data.result as SelfServiceRun)

        setExecutionTime(getTotalExecutionTime((data.result as SelfServiceRun).tasks));
      });

    fetch(`${API_URL}/selfServiceSectionsRuns/${taskId}/logs`)
      .then(res => res.json())
      .then((data) => {
        const logs = data.results.map(mapLog) as Array<Log>;
        setAddons([
          new DataLoggerAddon(logs),
          fitAddon,
        ])

        setTimeout(() => {
          fitAddon.fit();
        }, 1)
      });
  }, [taskId]);

  return (
    <>
      <div className="fixed size-full flex-col items-end justify-end pr-96">
        <div className="mb-16">
          {data &&
            <header className="flex w-full justify-between">
              <div className="flex items-center justify-center">
                <Link to="/self-service/run-history">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="size-6">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M9 15 3 9m0 0 6-6M3 9h12a6 6 0 0 1 0 12h-3" />
                  </svg>
                </Link>

                <h1 className="ml-6 text-3xl font-bold leading-tight tracking-tight text-gray-600">
                  {data.action_slug}
                </h1>
              </div>

              <div className="flex">
                <div className="mr-4 rounded-lg bg-cyan-400 px-4 py-2 text-xl font-bold tracking-wider text-white">
                  <p className="">{executionTime/1000}s</p>
                </div>
                
                <div className="rounded-lg bg-green-400 px-4 py-2 text-xl font-bold tracking-wider text-white">
                  <p className="">{data.status}</p>
                </div>
              </div>
            </header>
          }
        </div>

        <div className="h-full text-cyan-200">
          <XTerm
            className="h-full"
            addons={addons}
            options={{ cursorBlink: false }}
            style={{ width: '100%', height: "70%" }}
          />
        </div>
      </div>
    </>
  );
};