import { createRouter, createWebHistory } from 'vue-router'
import {DashboardRoutes} from "@/Pages/dashboard/index.js";
import MainView from "@/views/MainView.vue";
import AuthView from "@/views/AuthView.vue";
import {AuthRoutes} from "@/Pages/auth/index.js";


const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      component: MainView,
      children: [
        ...DashboardRoutes // Spread the array items directly here
      ]
    },
    {
      path: '/auth',
      component: AuthView,
      children: [
          ...AuthRoutes
      ]
    }
  ]
})

export default router