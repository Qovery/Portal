console.log('Executing script node get weather Paris');

for(let i = 0; i < 20; i++) {
  console.error(`Log error ${i}`);
}

fetch('https://wttr.in/Paris?format=j1')
  .then(res => res.json())
  .then(data => {
    console.log(`Currently it's ${data.current_condition.at(0).temp_C}°C in Paris.`);
    console.log('Finished!');
  });
