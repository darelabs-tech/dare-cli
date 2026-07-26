import { createApp } from 'vue';

const app = createApp({
  template: `<h1>{{project_name}} — {{stack_id}}</h1>`,
});

app.mount('#app');
