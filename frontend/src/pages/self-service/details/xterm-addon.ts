import { Terminal, IDisposable, ITerminalAddon } from '@xterm/xterm';
import { Log } from './details';

export class DataLoggerAddon implements ITerminalAddon {
  private _disposables: IDisposable[] = [];
  private data: Array<Log>;

  constructor(data: Array<Log>) {
    this.data = data
  }

  activate(terminal: Terminal): void {
    this._disposables.push(terminal.onData(d => console.log(d)));
    this.data.forEach(log => {
      const timePrefix = '\x1b[1;30m' + log.createdAt + '\x1b[37m' + " "
      if (log.isStderr) {
        terminal.writeln(timePrefix + '\x1b[1;31m' + log.message + '\x1b[37m');
      } else {
        terminal.writeln(timePrefix + log.message);
      }
    })
  }

  dispose(): void {
    this._disposables.forEach(d => d.dispose());
    this._disposables.length = 0;
  }
}
