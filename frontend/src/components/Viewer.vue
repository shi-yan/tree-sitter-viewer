<template>
  <div>
    <button @click="updateCode">Update</button>
    <select @change="onDirectionChanged">
      <option value="horizontal">Horizontal</option>
      <option value="vertical">Vertical</option>
    </select>
    <div ref="container" style="height: 50vh"></div>
    <div style="height: 50vh">
      <blocks-tree
        v-if="treeData.kind"
        :data="treeData"
        :horizontal="treeOrientation == '1'"
        :collapsable="true"
        :props="{
          label: 'label',
          expand: 'expand',
          children: 'children',
          key: 'id',
        }"
      >
        <template #node="{ data, context }">
          <div style="min-width: 80px; display: flex; flex-direction: row">
            <span class="tree-node-kind">{{ data.kind }}</span>
            <span class="tree-node-name" v-if="data.name">
              {{ data.name }}</span
            >
            <span class="tree-node-content" v-if="data.content">
              {{ data.content }}</span
            >
          </div>
        </template>
      </blocks-tree>
    </div>
  </div>
</template>

<script>
import * as monaco from "monaco-editor";
import { defineComponent, ref, reactive } from "vue";

export default {
  name: "Viewer",
  setup() {
    let treeOrientation = ref("1");
    let treeData = reactive({});

    const toggleSelect = (node, isSelected) => {
      isSelected
        ? selected.value.push(node.some_id)
        : selected.value.splice(selected.value.indexOf(node.some_id), 1);
      if (node.children && node.children.length) {
        node.children.forEach((ch) => {
          toggleSelect(ch, isSelected);
        });
      }
    };

    return {
      treeData,
      toggleSelect,
      treeOrientation,
      editor: null,
    };
  },
  props: {
    msg: String,
  },
  mounted: function () {
    this.$nextTick(function () {
      this.editor = monaco.editor.create(this.$refs["container"], {
        language: "cpp",
        theme: "vs-dark",
      });

      (async () => {
        const response = await fetch("/api/initial_code");
        let json = await response.json();
        console.log(json);

        if (json.code) {
          this.editor.setValue(json.code);
          if (json.code.length > 0) {
            this.updateCode();
          }
        }
      })();
    });
  },
  methods: {
    onDirectionChanged: function (event) {
      console.log(event.target.value);
      if (event.target.value === "vertical") {
        this.treeOrientation = 1;
      } else event.target.value === "horizontal";
      {
        this.treeOrientation = 0;
      }
    },
    postData: async function (url = "", data = {}) {
      // Default options are marked with *
      const response = await fetch(url, {
        method: "POST", // *GET, POST, PUT, DELETE, etc.
        mode: "cors", // no-cors, *cors, same-origin
        cache: "no-cache", // *default, no-cache, reload, force-cache, only-if-cached
        credentials: "same-origin", // include, *same-origin, omit
        headers: {
          "Content-Type": "application/json",
          // 'Content-Type': 'application/x-www-form-urlencoded',
        },
        redirect: "follow", // manual, *follow, error
        referrerPolicy: "no-referrer", // no-referrer, *no-referrer-when-downgrade, origin, origin-when-cross-origin, same-origin, strict-origin, strict-origin-when-cross-origin, unsafe-url
        body: JSON.stringify(data), // body data type must match "Content-Type" header
      });
      return response.json(); // parses JSON response into native JavaScript objects
    },
    updateCode: function () {
      if (this.editor) {
        let payload = { code: this.editor.getValue() };
        console.log(payload);
        let self = this;
        this.postData("/api/update_code", payload).then((result) => {
          console.log("result", result, self.treeData);
          Object.assign(self.treeData, result);
          console.log("assigned", self.treeData);
        });
      }
    },
  },
};
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
h3 {
  margin: 40px 0 0;
}
ul {
  list-style-type: none;
  padding: 0;
}
li {
  display: inline-block;
  margin: 0 10px;
}
a {
  color: #42b983;
}
</style>

<style>
.tree-node-name {
  border-radius: 5px;
  background: #73ad21;
  padding: 4px;
  margin: 4px;
}

.tree-node-content {
  color: #bb196a;
  font-weight: 600;
  font-style: italic;
  padding: 4px;
  margin: 4px;
}

.tree-node-kind {
  padding: 4px;
  margin: 4px;
}
</style>
