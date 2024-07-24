import { useEffect, useState } from "react";
import Subheader from "../../../components/Subheader";
import { API_URL } from "../../../config";
import { match } from 'ts-pattern'
import { useParams } from "react-router-dom";

type Log = {
  id: string;
  createdAt: string;
  isStderr: boolean;
  message: string;
}

type SelfServiceRun = {
  id: string;
  section_slug: string;
  action_slug: string;
  status: "SUCCESS"
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

  const [data, setData] = useState<SelfServiceRun | undefined>(undefined);
  const [logs, setLogs] = useState<Array<Log>>([]);
  useEffect(() => {
    fetch(`${API_URL}/selfServiceSections/${taskId}`)
      .then(res => res.json())
      .then((data) => {
        setData(data.result as SelfServiceRun)
      });

    fetch(`${API_URL}/selfServiceSectionsRuns/${taskId}/logs`)
      .then(res => res.json())
      .then((data) => {
        setLogs(data.results.reverse().map(mapLog) as Array<Log>);
      });
  }, [taskId]);

  return (
    <>
      <div className="fixed size-full flex-col items-end justify-end">
        <div className="mb-6">
          {data && <Subheader pageTitle={data?.action_slug} />}
        </div>

        <div className="mb-8 py-2">
          {data && (
            <div className="relative w-24 rounded-lg border border-green-600 bg-green-600 p-1 text-center text-white">{data.status}</div>
          )}
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