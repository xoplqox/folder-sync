import { createRouter, createWebHistory } from "vue-router";
import HeroView from "../views/HeroView.vue";
import TreeView from "../views/TreeView.vue";

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", name: "hero", component: HeroView },
    {
      path: "/drives/:name/:number",
      name: "tree",
      component: TreeView,
      props: true,
    },
  ],
});
