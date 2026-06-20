<template>
  <div class="process-chart">
    <h2>历史趋势</h2>
    <div class="chart-tabs">
      <button
        v-for="proc in processNames"
        :key="proc"
        :class="['tab-btn', { active: selectedProcess === proc }]"
        @click="selectedProcess = proc"
      >
        {{ proc }}
      </button>
    </div>
    <div class="chart-container">
      <Line v-if="chartData" :data="chartData" :options="chartOptions" />
      <div v-else class="no-data">暂无数据</div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend
} from 'chart.js'
import { Line } from 'vue-chartjs'

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend
)

const props = defineProps({
  processes: {
    type: Array,
    required: true
  },
  historyData: {
    type: Object,
    required: true
  }
})

const selectedProcess = ref(null)

const processNames = computed(() => {
  return props.processes.map(p => p.name)
})

watch(processNames, (names) => {
  if (names.length > 0 && !selectedProcess.value) {
    selectedProcess.value = names[0]
  }
  if (names.length > 0 && !names.includes(selectedProcess.value)) {
    selectedProcess.value = names[0]
  }
}, { immediate: true })

const chartData = computed(() => {
  if (!selectedProcess.value) return null
  const data = props.historyData[selectedProcess.value]
  if (!data || data.length === 0) return null

  const labels = data.map((_, i) => `-${(data.length - 1 - i) * 2}s`)
  
  return {
    labels,
    datasets: [
      {
        label: 'CPU 使用率 (%)',
        data: data.map(d => d.cpu_usage || 0),
        borderColor: '#667eea',
        backgroundColor: 'rgba(102, 126, 234, 0.1)',
        yAxisID: 'y',
        tension: 0.3,
        fill: true
      },
      {
        label: '内存占用 (MB)',
        data: data.map(d => d.memory_mb || 0),
        borderColor: '#f093fb',
        backgroundColor: 'rgba(240, 147, 251, 0.1)',
        yAxisID: 'y1',
        tension: 0.3,
        fill: true
      }
    ]
  }
})

const chartOptions = {
  responsive: true,
  maintainAspectRatio: false,
  interaction: {
    mode: 'index',
    intersect: false,
  },
  plugins: {
    legend: {
      position: 'top',
    },
    tooltip: {
      backgroundColor: 'rgba(0, 0, 0, 0.8)',
    }
  },
  scales: {
    y: {
      type: 'linear',
      display: true,
    position: 'left',
      title: {
        display: true,
        text: 'CPU (%)'
      },
      min: 0,
    },
    y1: {
      type: 'linear',
      display: true,
      position: 'right',
      title: {
        display: true,
        text: '内存 (MB)'
      },
      min: 0,
      grid: {
        drawOnChartArea: false,
      },
    },
  },
}
</script>

<style scoped>
.process-chart {
  background: white;
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.06);
}

.process-chart h2 {
  font-size: 18px;
  margin-bottom: 16px;
  color: #333;
}

.chart-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.tab-btn {
  padding: 6px 16px;
  background: #f0f0f0;
  border: none;
  border-radius: 20px;
  cursor: pointer;
  font-size: 13px;
  color: #666;
  transition: all 0.2s;
}

.tab-btn:hover {
  background: #e0e0e0;
}

.tab-btn.active {
  background: #667eea;
  color: white;
}

.chart-container {
  height: 300px;
  position: relative;
}

.no-data {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #999;
  font-size: 14px;
}
</style>
