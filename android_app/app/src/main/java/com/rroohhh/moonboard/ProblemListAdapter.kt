package com.rroohhh.moonboard

import android.view.LayoutInflater
import android.view.ViewGroup
import android.widget.TextView
import androidx.recyclerview.widget.RecyclerView

class ProblemListAdapter(val dataset_init: IntArray) :
    RecyclerView.Adapter<ProblemListAdapter.ViewHolder>() {
    var dataset: IntArray = dataset_init
        set(value) {
            notifyDataSetChanged()
            field = value
        }

    class ViewHolder(val textView: TextView) : RecyclerView.ViewHolder(textView)

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ViewHolder {
        val textView = LayoutInflater.from(parent.context)
            .inflate(R.layout.problem_view, parent, false) as TextView

        return ViewHolder(textView)
    }

    override fun onBindViewHolder(holder: ViewHolder, position: Int) {
        holder.textView.text = dataset[position].toString()
    }

    override fun getItemCount() = dataset.size
}