import { createApp } from 'vue'
import VueBlocksTree from 'vue3-blocks-tree';

import App from './App.vue'
let defaultoptions = {treeName:'blocks-tree'}

createApp(App).use(VueBlocksTree,defaultoptions).mount('#app')
