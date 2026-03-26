import { createRouter, createWebHistory } from 'vue-router'
import {DashboardRoutes} from "@/Pages/dashboard/index.js";
import MainView from "@/views/MainView.vue";


const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      component: MainView,
      children: [
        ...DashboardRoutes // Spread the array items directly here
      ]
    }
  ]
})

export default router