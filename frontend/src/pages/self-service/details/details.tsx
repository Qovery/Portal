import { useEffect, useState } from "react";
import Subheader from "../../../components/Subheader";
import { API_URL } from "../../../config";
import { match } from 'ts-pattern'

type Log = {
  id: string;
  createdAt: string;
  isStderr: boolean;
  message: string;
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
  const [data, setData] = useState({});
  const [logs, setLogs] = useState<Array<Log>>([]);
  useEffect(() => {
    fetch(`${API_URL}/selfServiceSections/63f65f2c-f045-4e38-aef8-fd15e11b6e50`)
      .then(res => res.json())
      .then((data) => {
        setData(data)
      });

    fetch(`${API_URL}/selfServiceSectionsRuns/63f65f2c-f045-4e38-aef8-fd15e11b6e50/logs`)
      .then(res => res.json())
      .then((data) => {
        setLogs(data.results.reverse().map(mapLog) as Array<Log>);
      });
  }, []);

  return (
    <>
      <div className="fixed size-full flex-col items-end justify-end">
        <div className="mb-6">
          <Subheader pageTitle="Details" />
        </div>

        <div>
          cc
        </div>

        <div className="h-2/3 overflow-scroll bg-slate-600 p-2 text-white">
          {
            logs.map(log => match(log)
              .with({ isStderr: true }, () => (
                <div className="text-red-200"><span className="mr-2 text-gray-400">{log.createdAt}</span> {log.message}</div>
              ))
              .with({ isStderr: false }, () => (
                <div><span className="mr-2 text-gray-400">{log.createdAt}</span> {log.message}</div>
              ))
              .exhaustive()
            )
          }
        </div>
      </div>
    </>
  );
};